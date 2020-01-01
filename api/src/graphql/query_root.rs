use crate::{
	db::helpers as DBHelper,
	graphql::context::Context,
	models::{CollectionMethod, Order, PostDeliveryOption},
	stripe::get_stripe,
};
use juniper::{graphql_value, FieldResult};

pub struct QueryRoot;
#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
	/// All orders
	fn orders(context : &Context) -> Vec<Order> {
		let orders = context.orders_handel();
		DBHelper::all(orders)
	}

	fn order(context : &Context, id : String) -> FieldResult<Option<Order>> {
		let orders = context.orders_handel();

		let id = match mongodb::oid::ObjectId::with_string(&id) {
			Ok(oid) => oid,
			Err(_) => {
				return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "INVALID_UID"
					}),
				))
			},
		};

		Ok(DBHelper::get(orders, id))
	}

	/// For an order, calculate the price to post the items to the user
	fn calculatePostage(context : &Context, id : String) -> FieldResult<Vec<PostDeliveryOption>> {
		let orders = context.orders_handel();

		let id = match mongodb::oid::ObjectId::with_string(&id) {
			Ok(oid) => oid,
			Err(_) => {
				return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "INVALID_UID"
					}),
				))
			},
		};

		let order : Order = match DBHelper::get(orders, id) {
			Some(o) => o,
			None => {
				return Err(juniper::FieldError::new(
					"The requested order was not found",
					graphql_value!({
						"type": "NOT_FOUND"
					}),
				))
			},
		};

		let postcode = match order.address {
			Some(addr) => addr.post_code,
			None => {
				return Err(juniper::FieldError::new(
					"This order does not have an address defined. This is likely because the Pickup option was selected",
					graphql_value!({
						"type": "NO_ADDRESS"
					}),
				))
			},
		};

		match PostDeliveryOption::get(order.quantity as u32, postcode as u32) {
			Ok(opts) => Ok(opts),
			Err(_) => Err(juniper::FieldError::new(
				"Quantity must be greater than 0",
				graphql_value!({
					"type": "INVALID_QUANTITY"
				}),
			)),
		}
	}

	/// Return the price of the order, excluding postage
	fn orderPrice(context : &Context, id : String) -> FieldResult<f64> {
		let orders = context.orders_handel();

		let id = match mongodb::oid::ObjectId::with_string(&id) {
			Ok(oid) => oid,
			Err(_) => {
				return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "INVALID_UID"
					}),
				))
			},
		};

		let order : Order = match DBHelper::get(orders, id) {
			Some(o) => o,
			None => {
				return Err(juniper::FieldError::new(
					"The requested order was not found",
					graphql_value!({
						"type": "NOT_FOUND"
					}),
				))
			},
		};

		let stripe_client = get_stripe();

		let pi = order.payment.unwrap().stripe.unwrap().pi;

		let price = match stripe::PaymentIntent::retrieve(&stripe_client, &pi) {
			Ok(pi) => pi.amount,
			_ => {
				return Err(juniper::FieldError::new(
					"Internal Error decoding Document from database",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
			},
		};

		Ok(f64::from(price as u32) / 100.0)
	}

	/// Return the price of the order, excluding postage
	fn getStripeCS(context : &Context, id : String) -> Option<String> {
		let orders = context.orders_handel();

		let id = match mongodb::oid::ObjectId::with_string(&id) {
			Ok(oid) => oid,
			Err(_) => return None,
		};

		let order : Order = match DBHelper::get(orders, id) {
			Some(o) => o,
			None => return None,
		};

		let stripe_client = get_stripe();

		let pi = order.payment.unwrap().stripe.unwrap().pi;

		let cs = match stripe::PaymentIntent::retrieve(&stripe_client, &pi) {
			Ok(pi) => pi.client_secret.unwrap(),
			_ => return None,
		};

		Some(cs)
	}

	/// Return the price of the order, excluding postage
	fn getOrderMethod(context : &Context, id : String) -> Option<String> {
		let orders = context.orders_handel();
		let id = match mongodb::oid::ObjectId::with_string(&id) {
			Ok(oid) => oid,
			Err(_) => return None,
		};

		let order : Order = match DBHelper::get(orders, id) {
			Some(o) => o,
			None => return None,
		};

		Some(String::from(match order.method {
			CollectionMethod::Pickup => "PICKUP",
			CollectionMethod::Post => "POST",
		}))
	}
}
