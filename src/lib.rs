#![no_std]
use atat_derive::AtatResp;

pub mod client;
pub mod digester;
pub mod general;
pub mod lora;
pub mod signal;
pub mod urc;

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct NoResponse;
