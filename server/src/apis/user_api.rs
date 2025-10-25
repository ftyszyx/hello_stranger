use entity::users;
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    username: String,
    exp: usize,
}

/// Register a new user.
#[endpoint]
pub async fn register(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.obtain::<DatabaseConnection>().unwrap();
    let reg_req: RegisterRequest = match req.parse_body().await {
        Ok(data) => data,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain(format!("Invalid request: {}", e)));
            return;
        }
    };

    // Check if user exists
    let existing_user = users::Entity::find()
        .filter(users::Column::Username.eq(reg_req.username.clone()))
        .one(db)
        .await
        .unwrap();

    if existing_user.is_some() {
        res.status_code(StatusCode::CONFLICT);
        res.render(Text::Plain("Username already exists".to_string()));
        return;
    }

    // Hash password
    let password_hash = bcrypt::hash(reg_req.password, bcrypt::DEFAULT_COST).unwrap();

    // Create user
    let new_user = users::ActiveModel {
        username: Set(reg_req.username),
        password_hash: Set(password_hash),
        ..Default::default()
    };

    // Save user
    let user = new_user.insert(db).await.unwrap();

    res.status_code(StatusCode::CREATED);
    res.render(Json(user));
}

/// Login for a user.
#[endpoint]
pub async fn login(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.obtain::<DatabaseConnection>().unwrap();
    let login_req: LoginRequest = match req.parse_body().await {
        Ok(data) => data,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain(format!("Invalid request: {}", e)));
            return;
        }
    };

    // Find user
    let user = match users::Entity::find()
        .filter(users::Column::Username.eq(login_req.username))
        .one(db)
        .await
        .unwrap()
    {
        Some(user) => user,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Text::Plain("Invalid username or password".to_string()));
            return;
        }
    };

    // Verify password
    let valid = bcrypt::verify(login_req.password, &user.password_hash).unwrap();
    if !valid {
        res.status_code(StatusCode::UNAUTHORIZED);
        res.render(Text::Plain("Invalid username or password".to_string()));
        return;
    }

    // Generate JWT
    let claims = Claims {
        username: user.username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("your-secret-key".as_ref()),
    )
    .unwrap();

    res.render(Json(AuthResponse { token }));
}

pub fn routes() -> Router {
    Router::with_path("auth")
        .post(register)
        .push(Router::with_path("login").post(login))
}
