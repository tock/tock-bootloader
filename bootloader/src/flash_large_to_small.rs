//! Map 512 byte pages onto larger hardware pages.
//!
//! The larger pages must be a multiple of 512 bytes, and the pages must be
//! aligned.

use core::cell::Cell;
use core::ops::{Index, IndexMut};

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::ReturnCode;

pub struct FiveTwelvePage(pub [u8; 512 as usize]);
impl Default for FiveTwelvePage {
    fn default() -> Self {
        Self {
            0: [0; 512 as usize],
        }
    }
}

impl Index<usize> for FiveTwelvePage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for FiveTwelvePage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for FiveTwelvePage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// This module is either waiting to do something, or handling a read/write.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Idle,
    Read { page_number: usize },
    Write { page_number: usize },
    Erase { page_number: usize },
}

pub struct FlashLargeToSmall<'a, Flarge: hil::flash::Flash + 'static> {
    flash_large: &'a Flarge,
    client: OptionalCell<&'static dyn hil::flash::Client<FlashLargeToSmall<'static, Flarge>>>,
    pagebuffer: TakeCell<'static, Flarge::Page>,

    client_pagebuffer: TakeCell<'static, FiveTwelvePage>,

    state: Cell<State>,
}

impl<'a, Flarge: hil::flash::Flash> FlashLargeToSmall<'a, Flarge> {
    pub fn new(
        flash_large: &'a Flarge,
        buffer: &'static mut Flarge::Page,
    ) -> FlashLargeToSmall<'a, Flarge> {
        FlashLargeToSmall {
            flash_large: flash_large,
            client: OptionalCell::empty(),
            pagebuffer: TakeCell::new(buffer),
            client_pagebuffer: TakeCell::empty(),
            state: Cell::new(State::Idle),
        }
    }

    fn get_large_page_index_offset(&self, small_page_index: usize) -> (usize, usize) {
        let large_size = self.pagebuffer.map_or(4096, |page| page.as_mut().len());
        let multiplier = large_size / 512;
        let large_index_start = small_page_index / multiplier;
        let large_index_offset = small_page_index % multiplier;
        (large_index_start, large_index_offset)
    }
}

impl<'a, C: hil::flash::Client<Self>, Flarge: hil::flash::Flash> hil::flash::HasClient<'static, C>
    for FlashLargeToSmall<'static, Flarge>
{
    fn set_client(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl<'a, Flarge: hil::flash::Flash> hil::flash::Flash for FlashLargeToSmall<'a, Flarge> {
    type Page = FiveTwelvePage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        // Translate to the large page we need to read.
        let (index, _) = self.get_large_page_index_offset(page_number);

        // Save the buffer to read into.
        self.client_pagebuffer.replace(buf);

        // Call the underlying flash layer.
        self.pagebuffer.take().map_or_else(
            || Err((ReturnCode::FAIL, self.client_pagebuffer.take().unwrap())),
            |page| {
                self.state.set(State::Read { page_number });
                self.flash_large.read_page(index, page).map_err(|e| {
                    self.pagebuffer.replace(e.1);
                    (ReturnCode::FAIL, self.client_pagebuffer.take().unwrap())
                })
            },
        )
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        // Translate to the large page we need to read.
        let (index, _) = self.get_large_page_index_offset(page_number);

        // Save the buffer to write from.
        self.client_pagebuffer.replace(buf);

        // Call the underlying flash layer to read the original large page.
        self.pagebuffer.take().map_or_else(
            || Err((ReturnCode::FAIL, self.client_pagebuffer.take().unwrap())),
            |page| {
                self.state.set(State::Write { page_number });
                self.flash_large.read_page(index, page).map_err(|e| {
                    self.pagebuffer.replace(e.1);
                    (ReturnCode::FAIL, self.client_pagebuffer.take().unwrap())
                })
            },
        )
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        // Translate to the large page we need to read.
        let (index, _) = self.get_large_page_index_offset(page_number);

        // Call the underlying flash layer to read the original large page.
        self.pagebuffer.take().map_or(ReturnCode::FAIL, |page| {
            self.state.set(State::Erase { page_number });
            self.flash_large.read_page(index, page).map_or_else(
                |e| {
                    self.pagebuffer.replace(e.1);
                    ReturnCode::FAIL
                },
                |_| ReturnCode::SUCCESS,
            )
        })
    }
}

impl<Flarge: hil::flash::Flash> hil::flash::Client<Flarge> for FlashLargeToSmall<'_, Flarge> {
    fn read_complete(&self, pagebuffer: &'static mut Flarge::Page, _error: hil::flash::Error) {
        match self.state.get() {
            State::Read { page_number } => {
                // Just need to read from the larger page into the smaller page
                // buffer.
                let (_, large_page_offset) = self.get_large_page_index_offset(page_number);
                let large_page_byte_offset = 512 * large_page_offset;

                self.client.map(|client| {
                    self.client_pagebuffer.take().map(move |smpage| {
                        for i in 0..512 {
                            smpage[i] = pagebuffer.as_mut()[large_page_byte_offset + i];
                        }

                        self.pagebuffer.replace(pagebuffer);

                        client.read_complete(smpage, hil::flash::Error::CommandComplete);
                    });
                });
            }

            State::Write { page_number } => {
                // Need to copy the new data from the small page into the larger
                // page.
                let (large_page_index, large_page_offset) =
                    self.get_large_page_index_offset(page_number);
                let large_page_byte_offset = 512 * large_page_offset;

                self.client_pagebuffer.map(move |smpage| {
                    for i in 0..512 {
                        pagebuffer.as_mut()[large_page_byte_offset + i] = smpage[i];
                    }

                    let _ = self.flash_large.write_page(large_page_index, pagebuffer);
                });
            }

            State::Erase { page_number } => {
                // Need to set the smaller page area with all 1s.
                let (large_page_index, large_page_offset) =
                    self.get_large_page_index_offset(page_number);
                let large_page_byte_offset = 512 * large_page_offset;

                for i in 0..512 {
                    pagebuffer.as_mut()[large_page_byte_offset + i] = 1;
                }

                let _ = self.flash_large.write_page(large_page_index, pagebuffer);
            }

            _ => {}
        }
    }

    fn write_complete(&self, pagebuffer: &'static mut Flarge::Page, _error: hil::flash::Error) {
        match self.state.get() {
            State::Write { page_number: _ } => {
                self.client.map(|client| {
                    self.client_pagebuffer.take().map(move |smpage| {
                        self.pagebuffer.replace(pagebuffer);

                        client.write_complete(smpage, hil::flash::Error::CommandComplete);
                    });
                });
            }

            State::Erase { page_number: _ } => {
                self.pagebuffer.replace(pagebuffer);
                self.client.map(|client| {
                    client.erase_complete(hil::flash::Error::CommandComplete);
                });
            }

            _ => {}
        }
    }

    fn erase_complete(&self, _error: hil::flash::Error) {}
}
