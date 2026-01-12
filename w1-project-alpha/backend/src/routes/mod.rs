use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::handlers::*;
use crate::repositories::Repositories;

pub fn create_routes(repositories: Repositories) -> Router {
    Router::new()
        // Ticket routes
        .route("/api/tickets", get(get_tickets).post(create_ticket))
        .route(
            "/api/tickets/:id",
            get(get_ticket)
                .put(update_ticket)
                .delete(delete_ticket),
        )
        .route("/api/tickets/:id/toggle", patch(toggle_ticket_completed))
        .route(
            "/api/tickets/:ticket_id/tags/:tag_id",
            post(add_tag_to_ticket).delete(remove_tag_from_ticket),
        )
        // Tag routes
        .route("/api/tags", get(get_tags).post(create_tag))
        .route(
            "/api/tags/:id",
            get(get_tag)
                .put(update_tag)
                .delete(delete_tag),
        )
        .with_state(repositories)
}
