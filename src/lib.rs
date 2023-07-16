#![no_std]
use atat_derive::AtatResp;

pub mod general;
pub mod lora;
pub mod client;
pub mod digester;
pub mod urc;

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct NoResponse;
