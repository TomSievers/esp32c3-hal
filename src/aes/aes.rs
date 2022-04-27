use core::marker::PhantomData;

use cipher::{BlockBackend, ParBlocksSizeUser, BlockSizeUser, consts::{U16, U1}, generic_array::ArrayLength};

pub struct Aes<S : 'static + ArrayLength<u8>> {

    _size : PhantomData<S>
}

impl<S : 'static + ArrayLength<u8>> BlockBackend for Aes<S> {
    fn proc_block(&mut self, mut block: cipher::inout::InOut<'_, '_, cipher::Block<Self>>) {

        let input_ptr = block.get_in().as_ptr() as usize;

        let in_channel : [u32; 3] = [
            0b1,
            input_ptr as u32,
            0
        ];

        
        todo!()
    }
}

impl<S : 'static + ArrayLength<u8>> ParBlocksSizeUser for Aes<S> {
    type ParBlocksSize = U1;
}

impl<S : 'static + ArrayLength<u8>> BlockSizeUser for Aes<S> {
    type BlockSize = S;
}