#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use regex::Regex;
use std::borrow::Cow;
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// SwapStatus Enum
#[derive(candid::CandidType, Deserialize, Serialize, Clone)]
enum SwapStatus {
    Pending,
    Accepted,
    Rejected,
}

// User Struct
#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
    phone_number: String,
    email: String,
    created_at: u64,
}

// KenyanShillings Struct (formerly Book)
#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct KenyanShillings {
    id: u64,
    user_id: u64,
    title: String,
    author: String,
    description: String,
    created_at: u64,
}

// SwapRequest Struct
#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct SwapRequest {
    id: u64,
    kenyan_shillings_id: u64, // Changed from book_id
    requested_by_id: u64,
    status: SwapStatus,
    created_at: u64,
}

// Feedback Struct
#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct Feedback {
    id: u64,
    user_id: u64,
    swap_request_id: u64,
    rating: u8,
    comment: String,
    created_at: u64,
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static USERS_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );

    static KENYAN_SHILLINGS_STORAGE: RefCell<StableBTreeMap<u64, KenyanShillings, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );

    static SWAP_REQUESTS_STORAGE: RefCell<StableBTreeMap<u64, SwapRequest, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );

    static FEEDBACK_STORAGE: RefCell<StableBTreeMap<u64, Feedback, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for KenyanShillings {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for KenyanShillings {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for SwapRequest {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for SwapRequest {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Feedback {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Feedback {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Payloads Definitions

// User Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct UserPayload {
    name: String,
    phone_number: String,
    email: String,
}

// KenyanShillings Payload (formerly Book)
#[derive(candid::CandidType, Deserialize, Serialize)]
struct KenyanShillingsPayload {
    user_id: u64,
    title: String,
    author: String,
    description: String,
}

// SwapRequest Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct SwapRequestPayload {
    kenyan_shillings_id: u64, // Changed from book_id
    requested_by_id: u64,
}

// Feedback Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct FeedbackPayload {
    user_id: u64,
    swap_request_id: u64,
    rating: u8,
    comment: String,
}

// Functions

#[ic_cdk::update]
fn create_user_profile(payload: UserPayload) -> Result<User, Error> {
    // Validate the payload to ensure that the required fields are present
    if payload.name.is_empty() || payload.phone_number.is_empty() || payload.email.is_empty() {
        return Err(Error::EmptyFields {
            msg: "All fields are required".to_string(),
        });
    }

    // Validate the payload to ensure that the email format is correct
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(Error::InvalidEmail {
            msg: "Ensure the email address is of the correct format".to_string(),
        });
    }

    // Ensure email address uniqueness
    let email_exists = USERS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, user)| user.email == payload.email)
    });

    if email_exists {
        return Err(Error::AlreadyExists {
            msg: "Email already exists".to_string(),
        });
    }

    // Validate the payload to ensure that the phone number format is correct and is 10 digits
    let phone_number_regex = Regex::new(r"^[0-9]{10}$").unwrap();
    if !phone_number_regex.is_match(&payload.phone_number) {
        return Err(Error::InvalidPhoneNumber {
            msg: "Ensure the phone number is of the correct format".to_string(),
        });
    }

    // Generate a new unique ID for the user
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    // Create the user profile
    let user_profile = User {
        id,
        name: payload.name,
        phone_number: payload.phone_number,
        email: payload.email,
        created_at: time(),
    };

    // Store the new user profile in the USERS_STORAGE
    USERS_STORAGE.with(|storage| storage.borrow_mut().insert(id, user_profile.clone()));

    Ok(user_profile)
}

// Function to get a user profile
#[ic_cdk::query]
fn get_user_profile(user_id: u64) -> Result<User, String> {
    // Ensure that the user exists
    let user = USERS_STORAGE.with(|storage| storage.borrow().get(&user_id));
    match user {
        Some(user) => Ok(user.clone()),
        None => Err("User does not exist".to_string()),
    }
}

// Function to update a user profile
#[ic_cdk::update]
fn update_user_profile(user_id: u64, payload: UserPayload) -> Result<User, String> {
    // Ensure that the user exists
    let user = USERS_STORAGE.with(|storage| storage.borrow().get(&user_id));
    match user {
        Some(user) => {
            // Validate the payload to ensure that the required fields are present
            if payload.name.is_empty() || payload.phone_number.is_empty() || payload.email.is_empty() {
                return Err("All fields are required".to_string());
            }

            // Validate the user id to ensure it exists
            if user_id != user.id {
                return Err("User does not exist".to_string());
            }

            // Validate the payload to ensure that the email format is correct
            let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
            if !email_regex.is_match(&payload.email) {
                return Err("Ensure the email address is of the correct format".to_string());
            }

            // Ensure email address uniqueness
            let email_exists = USERS_STORAGE.with(|storage| {
                storage
                    .borrow()
                    .iter()
                    .any(|(_, user)| user.email == payload.email && user.id != user_id)
            });

            if email_exists {
                return Err("Email already exists".to_string());
            }

            // Validate the payload to ensure that the phone number format is correct and is 10 digits
            let phone_number_regex = Regex::new(r"^[0-9]{10}$").unwrap();
            if !phone_number_regex.is_match(&payload.phone_number) {
                return Err("Ensure the phone number is of the correct format".to_string());
            }

            // Update the user profile
            let updated_user = User {
                id: user_id,
                name: payload.name,
                phone_number: payload.phone_number,
                email: payload.email,
                created_at: user.created_at,
            };

            // Store the updated user profile in the USERS_STORAGE
            USERS_STORAGE.with(|storage| storage.borrow_mut().insert(user_id, updated_user.clone()));

            Ok(updated_user)
        }
        None => Err("User does not exist".to_string()),
    }
}

// Error Handling
#[derive(Debug, candid::CandidType, Deserialize)]
enum Error {
    EmptyFields { msg: String },
    InvalidEmail { msg: String },
    InvalidPhoneNumber { msg: String },
    AlreadyExists { msg: String },
    UserNotFound { msg: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyFields { msg }
            | Error::InvalidEmail { msg }
            | Error::InvalidPhoneNumber { msg }
            | Error::AlreadyExists { msg }
            | Error::UserNotFound { msg } => write!(f, "{}", msg),
        }
    }
}

// need this to generate candid
ic_cdk::export_candid!();