use crate::handlers::*;
use crate::{
    chat::CreateChat,
    messages::{CreateMessage, ListMessages},
    user::{CreateUser, SigninUser},
    AuthOutput, ChatUser,
};
use axum::Router;
use chat_core::{Chat, ChatType, Message, User, WorkSpace};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;
pub trait OpenApiRouter {
    fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
    paths(
        signup_handler,
        signin_handler,
        list_chat_handler,
        get_chat_handler,
        create_chat_handler,
        send_message_handler,
        list_message_handler,
        list_workspace_users_handler,
    ),
    components(schemas( CreateUser,User, CreateChat,Chat, ChatType, AuthOutput, ChatUser, CreateMessage,SigninUser, Message, WorkSpace,ListMessages)),
    modifiers(&SecurityAddon),
    tags(
        (name="chat", description="Chat related operations")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            );
        }
    }
}

impl OpenApiRouter for Router<AppState> {
    fn openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    }
}
