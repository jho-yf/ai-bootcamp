use sqlx::PgPool;

pub mod tag_repository;
pub mod ticket_repository;

pub use tag_repository::TagRepository;
pub use ticket_repository::TicketRepository;

#[derive(Clone)]
pub struct Repositories {
    pub ticket: TicketRepository,
    pub tag: TagRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            ticket: TicketRepository::new(pool.clone()),
            tag: TagRepository::new(pool),
        }
    }
}
