use crate::db::PrimaryDb;
use juniper::Context as JuniperContext;
use mongodb::{coll::Collection, db::ThreadedDatabase};

pub struct Context {
	pub connection : PrimaryDb,
}

impl Context {
	pub fn orders_handel(&self) -> Collection { self.connection.collection("orders") }
}

impl JuniperContext for Context {}
