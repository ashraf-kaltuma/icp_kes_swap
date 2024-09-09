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
    // Validate the payload
    validate_user_payload(&payload)?;

    // Ensure email address uniqueness
    if email_exists(&payload.email) {
        return Err(Error::AlreadyExists {
            msg: "Email already exists".to_string(),
        });
    }

    // Generate a new unique ID for the user
    let id = increment_id_counter()?;

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
fn get_user_profile(user_id: u64) -> Result<User, Error> {
    USERS_STORAGE
        .with(|storage| storage.borrow().get(&user_id))
        .ok_or_else(|| Error::UserNotFound {
            msg: "User does not exist".to_string(),
        })
        .map(|user| user.clone())
}

// Function to update a user profile
#[ic_cdk::update]
fn update_user_profile(user_id: u64, payload: UserPayload) -> Result<User, Error> {
    let mut user = USERS_STORAGE
        .with(|storage| storage.borrow().get(&user_id))
        .ok_or_else(|| Error::UserNotFound {
            msg: "User does not exist".to_string(),
        })?
        .clone();

    // Validate the payload
    validate_user_payload(&payload)?;

    // Ensure email address uniqueness, excluding the current user
    if email_exists_excluding(&payload.email, user_id) {
        return Err(Error::AlreadyExists {
            msg: "Email already exists".to_string(),
        });
    }

    // Update the user profile
    user.name = payload.name;
    user.phone_number = payload.phone_number;
    user.email = payload.email;

    USERS_STORAGE.with(|storage| storage.borrow_mut().insert(user_id, user.clone()));

    Ok(user)
}

// Utility functions

fn validate_user_payload(payload: &UserPayload) -> Result<(), Error> {
    if payload.name.is_empty() || payload.phone_number.is_empty() || payload.email.is_empty() {
        return Err(Error::EmptyFields {
            msg: "All fields are required".to_string(),
        });
    }

    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(Error::InvalidEmail {
            msg: "Ensure the email address is of the correct format".to_string(),
        });
    }

    let phone_number_regex = Regex::new(r"^[0-9]{10}$").unwrap();
    if !phone_number_regex.is_match(&payload.phone_number) {
        return Err(Error::InvalidPhoneNumber {
            msg: "Ensure the phone number is of the correct format".to_string(),
        });
    }

    Ok(())
}

fn email_exists(email: &str) -> bool {
    USERS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, user)| user.email == email)
    })
}

fn email_exists_excluding(email: &str, exclude_id: u64) -> bool {
    USERS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, user)| user.email == email && user.id != exclude_id)
    })
}

#[ic_cdk::query]
fn search_user(query: String) -> Result<Vec<User>, Error> {
    // Check if the query is an email
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if email_regex.is_match(&query) {
        let users = USERS_STORAGE.with(|storage| {
            storage
                .borrow()
                .iter()
                .filter(|(_, user)| user.email == query)
                .map(|(_, user)| user.clone())
                .collect::<Vec<User>>()
        });

        if users.is_empty() {
            return Err(Error::UserNotFound {
                msg: "No user found with the provided email".to_string(),
            });
        }

        return Ok(users);
    }

    // Check if the query is a phone number
    let phone_number_regex = Regex::new(r"^[0-9]{10}$").unwrap();
    if phone_number_regex.is_match(&query) {
        let users = USERS_STORAGE.with(|storage| {
            storage
                .borrow()
                .iter()
                .filter(|(_, user)| user.phone_number == query)
                .map(|(_, user)| user.clone())
                .collect::<Vec<User>>()
        });

        if users.is_empty() {
            return Err(Error::UserNotFound {
                msg: "No user found with the provided phone number".to_string(),
            });
        }

        return Ok(users);
    }

    Err(Error::InvalidQuery {
        msg: "Query must be a valid email or phone number".to_string(),
    })
}

#[ic_cdk::update]
fn create_swap_request(payload: SwapRequestPayload) -> Result<SwapRequest, Error> {
    // Ensure that the KenyanShillings item exists
    let kenyan_shillings = KENYAN_SHILLINGS_STORAGE.with(|storage| {
        storage.borrow().get(&payload.kenyan_shillings_id)
    }).ok_or_else(|| Error::NotFound {
        msg: "KenyanShillings item not found".to_string(),
    })?;

    // Ensure that the requesting user is not the owner of the KenyanShillings item
    if kenyan_shillings.user_id == payload.requested_by_id {
        return Err(Error::Unauthorized {
            msg: "Cannot create a swap request for your own item".to_string(),
        });
    }

    // Generate a new unique ID for the swap request
    let id = increment_id_counter()?;

    // Create the swap request
    let swap_request = SwapRequest {
        id,
        kenyan_shillings_id: payload.kenyan_shillings_id,
        requested_by_id: payload.requested_by_id,
        status: SwapStatus::Pending,
        created_at: time(),
    };

    // Store the new swap request in the SWAP_REQUESTS_STORAGE
    SWAP_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, swap_request.clone()));

    Ok(swap_request)
}




fn increment_id_counter() -> Result<u64, Error> {
    ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .map_err(|_| Error::IncrementCounterFailed {
            msg: "Failed to increment the ID counter".to_string(),
        })
}

// Error Handling
#[derive(Debug, candid::CandidType, Deserialize)]
enum Error {
    EmptyFields { msg: String },
    InvalidEmail { msg: String },
    InvalidPhoneNumber { msg: String },
    InvalidQuery { msg: String }, // Add this variant
    AlreadyExists { msg: String },
    UserNotFound { msg: String },
    NotFound { msg: String },
    Unauthorized { msg: String },
    IncrementCounterFailed { msg: String },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyFields { msg }
            | Error::InvalidEmail { msg }
            | Error::InvalidPhoneNumber { msg }
            | Error::InvalidQuery { msg }
            | Error::AlreadyExists { msg }
            | Error::UserNotFound { msg }
            | Error::NotFound { msg }
            | Error::Unauthorized { msg }
            | Error::IncrementCounterFailed { msg } => write!(f, "{}", msg),
        }
    }
}


// Generate the candid interface
ic_cdk::export_candid!();
