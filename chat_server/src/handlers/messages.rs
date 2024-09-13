use crate::{
    error::AppError,
    messages::{CreateMessage, ListMessages},
    AppState, ChatFile,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{self},
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;
use tokio::fs::{self};
use tracing::{info, warn};

pub(crate) async fn send_message_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<u64>,
    Extension(user): Extension<User>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let msg = state
        .create_message(input, chat_id as _, user.id as _)
        .await?;
    Ok(Json(msg))
}

pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let msgs = state.list_messages(input).await?;
    Ok(Json(msgs))
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut mutpart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];
    while let Some(field) = mutpart.next_field().await.unwrap() {
        let filename = field.file_name().map(|s| s.to_string());
        let (Some(filename), Ok(data)) = (&filename, field.bytes().await) else {
            warn!("Failed to read multipart file field: {:?}", filename);
            continue;
        };
        let file = ChatFile::new(ws_id as _, filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("File already exists: {:?}", path);
            continue;
        } else {
            fs::create_dir_all(path.parent().unwrap())
                .await
                .expect("file path parent should exist");
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }
    Ok(Json(files))
}

pub(crate) async fn download_file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(u64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id as i64 {
        return Err(AppError::NotFound(
            "file not found or you don't have permission access".to_string(),
        ));
    }
    let base_dir = &state.config.server.base_dir;
    let path = base_dir.join(ws_id.to_string()).join(path);
    info!("Download file: {:?}", path);
    if !path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path).await?;
    // let file = File::open(path).await?;

    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
    Ok((headers, body))
}
