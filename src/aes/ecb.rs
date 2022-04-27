//! Implementation of hardware AES-ECB
use cipher::{BlockEncrypt, BlockSizeUser, BlockClosure, consts::{U16}};

pub struct Encryptor {

}

impl BlockEncrypt for Encryptor {
    fn encrypt_with_backend(&self, f: impl BlockClosure<BlockSize = Self::BlockSize>) {
        
    }
}

impl BlockSizeUser for Encryptor {
    type BlockSize = U16;

}