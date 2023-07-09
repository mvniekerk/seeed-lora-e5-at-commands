#![no_std]
use atat_derive::AtatResp;

pub mod general;
pub mod lora;

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct NoResponse;
