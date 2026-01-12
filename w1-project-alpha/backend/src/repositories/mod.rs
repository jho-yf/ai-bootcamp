use sqlx::PgPool;

pub mod ticket_repository;
pub mod tag_repository;

pub use ticket_repository::TicketRepository;
pub use tag_repository::TagRepository;

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
