use crate::{
	db::helpers as DBHelper,
	graphql::context::Context,
	models::{CollectionMethod, Order, PostDeliveryOption},
	stripe::get_stripe,
};
use juniper::{graphql_value, FieldResult};
use mongodb::{oid::ObjectId, Bson};

const POST_OPTION : &str = "AUS_PARCEL_REGULAR_PACKAGE_SMALL";
const SCARVE_PRICE : u64 = 1500;

pub struct MutationRoot;
#[juniper::object(
    Context = Context
)]
impl MutationRoot {
	/// Take in the details of a user, how they would like to receive their
	/// order and possibly their address.
	fn newOrder(
		context : &Context,
		name : String,
		quantity : i32,
		email : String,
		address_apt : Option<String>,
		address_street : Option<String>,
		address_town : Option<String>,
		address_state : Option<String>,
		address_post_code : Option<i32>,
		delivery_method : CollectionMethod,
	) -> FieldResult<Option<Order>> {
		let stripe_client = get_stripe();

		if quantity < 1 {
			return Err(juniper::FieldError::new(
				"Quantity must be greater than 0",
				graphql_value!({
					"type": "NO_WHATEVER"
				}),
			));
		};
		let orders = context.orders_handel();

		let result = orders
			.insert_one(
				doc! {
				"quantity" => quantity,
				"user" => {
					"name" => &name,
					"email" => &email,
				},
				"method" => match delivery_method {
					CollectionMethod::Pickup => 1,
					CollectionMethod::Post => 0,
				} as i32,
				},
				None,
			)
			.unwrap();

		match delivery_method {
			CollectionMethod::Post => {
				orders
					.update_one(
						doc! {"_id" => result.inserted_id.clone().expect("Inserted ID not found") },
						doc! {
							"$set" => {
								"address" => {
									"apartment" => match address_apt {Some(a) => a, None => "".to_string()},
									"street" => match address_street{Some(a) => a, None => "".to_string()},
									"town" => match address_town{Some(a) => a, None => "".to_string()},
									"state" => match address_state{Some(a) => a, None => "".to_string()},
									"post_code" => match address_post_code{Some(a) => a, None => 0},
								}
							}
						},
						None,
					)
					.expect("Update of delivery method failed");
			},
			_ => {},
		};

		let id = result.inserted_id.unwrap_or(Bson::ObjectId(
			ObjectId::new().expect("Converting mongoID to juniperID failed"),
		));

		let id = id.as_object_id().expect("Unwrap string").to_string();

		let post_price : u64 = match delivery_method {
			CollectionMethod::Post => {
				let dopts = match PostDeliveryOption::get(
					quantity as u32,
					address_post_code.unwrap() as u32,
				) {
					Ok(opts) => opts,
					Err(_) => {
						return Err(juniper::FieldError::new(
							"Quantity must be greater than 0",
							graphql_value!({
								"type": "INVALID_QUANTITY"
							}),
						))
					},
				};

				let opt = dopts
					.into_iter()
					.filter(|opt : &PostDeliveryOption| opt.code == POST_OPTION)
					.collect::<Vec<PostDeliveryOption>>();

				let opt : &PostDeliveryOption = match opt.first() {
					Some(o) => o,
					_ => {
						return Err(juniper::FieldError::new(
							"User has not selected a valid postage option",
							graphql_value!({
								"type": "INVALID_QUANTITY"
							}),
						))
					},
				};

				(opt.price * f64::from(100)) as u64
			},
			_ => 0,
		};

		let mut params = stripe::PaymentIntentCreateParams::new(
			SCARVE_PRICE * quantity as u64 + post_price,
			stripe::Currency::AUD,
		);

		let desc = format!(
			"{}: Scarves x{} for {}",
			name,
			&quantity,
			match delivery_method {
				CollectionMethod::Pickup => "Pickup",
				CollectionMethod::Post => "Postage",
			}
		);

		params.description = Some(&desc);

		let cus = String::from(&email);
		let mut meta = stripe::Metadata::new();
		meta.insert("email".to_string(), cus);
		meta.insert("quantity".to_string(), quantity.to_string());
		params.metadata = Some(meta);

		let pi = match stripe::PaymentIntent::create(&stripe_client, params) {
			Ok(pi) => {
				orders
					.update_one(
						doc! {"_id" => ObjectId::with_string(&id).unwrap()},
						doc! {
							"$set" => {
								"payment" => {
									"stripe" => {
										"pi" => pi.id.as_str(),
									}
								}
							}
						},
						None,
					)
					.expect("Updating Order failed");
				pi
			},
			_ => {
				return Err(juniper::FieldError::new(
					"Failed to create payment intent",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
			},
		};

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

		let mut order = match DBHelper::get::<Order>(orders, id) {
			Some(order) => order,
			_ => {
				return Err(juniper::FieldError::new(
					"Order dissapeared after creation",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
			},
		};

		let mut payment = &mut order
			.payment
			.as_mut()
			.expect("Unwrapping mutable payment failed");

		let mut stripe = &mut payment
			.stripe
			.as_mut()
			.expect("Unwrapping mutable stripe payment failed");

		stripe.client_secret = Some(pi.client_secret.expect("Unwrapping client secret failed"));

		Ok(Some(order))
	}

	/// Set the requested postage method from the user. Once this is done the
	/// order is practically finalized and just needs to be paid for.
	fn setPostage(context : &Context, id : String, code : String) -> FieldResult<Order> {
		let stripe_client = get_stripe();
		let orders = context.orders_handel();

		orders
			.update_one(
				doc! {
				"_id" => match mongodb::oid::ObjectId::with_string(&id) {
					Ok(oid) => oid,
					Err(_) => return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
				}},
				doc! {
					"$set" => {
						"postage" => {
							"code" => &code,
						}
					}
				},
				None,
			)
			.unwrap();

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

		let pi = order.payment.clone().unwrap().stripe.unwrap().pi;

		let q = order.quantity.clone();

		let dopts = match PostDeliveryOption::get(
			order.quantity as u32,
			order.address.clone().unwrap().post_code as u32,
		) {
			Ok(opts) => opts,
			Err(_) => {
				return Err(juniper::FieldError::new(
					"Quantity must be greater than 0",
					graphql_value!({
						"type": "INVALID_QUANTITY"
					}),
				))
			},
		};

		let opt = dopts
			.into_iter()
			.filter(|opt : &PostDeliveryOption| opt.code == code)
			.collect::<Vec<PostDeliveryOption>>();

		let opt : &PostDeliveryOption = match opt.first() {
			Some(o) => o,
			_ => {
				return Err(juniper::FieldError::new(
					"User has not selected a valid postage option",
					graphql_value!({
						"type": "INVALID_QUANTITY"
					}),
				))
			},
		};

		let price = (opt.price * f64::from(100)) as u64;

		match stripe::PaymentIntent::update(
			&stripe_client,
			&pi,
			stripe::PaymentIntentUpdateParams {
				amount :                  Some(SCARVE_PRICE * q as u64 + price),
				application_fee_amount :  None,
				currency :                None,
				customer :                None,
				description :             None,
				metadata :                None,
				receipt_email :           None,
				save_source_to_customer : None,
				shipping :                None,
				source :                  None,
				transfer_group :          None,
			},
		) {
			Ok(p) => {},
			_ => {},
		};

		Ok(order)
	}
}
