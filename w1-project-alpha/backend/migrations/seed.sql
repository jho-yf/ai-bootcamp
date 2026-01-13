-- Seed data for Project Alpha
-- This file contains sample data for testing and development
-- Run this file after running the initial migration

-- Insert 10 tags
INSERT INTO tags (id, name, color, created_at) VALUES
    ('11111111-1111-1111-1111-111111111111', 'urgent', '#FF0000', NOW()),
    ('22222222-2222-2222-2222-222222222222', 'bug', '#FF5733', NOW()),
    ('33333333-3333-3333-3333-333333333333', 'feature', '#33FF57', NOW()),
    ('44444444-4444-4444-4444-444444444444', 'documentation', '#3357FF', NOW()),
    ('55555555-5555-5555-5555-555555555555', 'enhancement', '#FF33F5', NOW()),
    ('66666666-6666-6666-6666-666666666666', 'question', '#F5FF33', NOW()),
    ('77777777-7777-7777-7777-777777777777', 'help-wanted', '#33FFF5', NOW()),
    ('88888888-8888-8888-8888-888888888888', 'backend', '#FF8C33', NOW()),
    ('99999999-9999-9999-9999-999999999999', 'frontend', '#8C33FF', NOW()),
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'testing', '#33FF8C', NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert 50 tickets
INSERT INTO tickets (id, title, description, completed, created_at, updated_at) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Fix login authentication bug', 'Users are unable to login with their email addresses. Need to investigate the authentication flow.', false, NOW() - INTERVAL '10 days', NOW() - INTERVAL '10 days'),
    ('10000000-0000-0000-0000-000000000002', 'Implement dark mode theme', 'Add dark mode support to the application with user preference persistence.', false, NOW() - INTERVAL '9 days', NOW() - INTERVAL '9 days'),
    ('10000000-0000-0000-0000-000000000003', 'Update API documentation', 'Update the API documentation with latest endpoints and examples.', true, NOW() - INTERVAL '8 days', NOW() - INTERVAL '7 days'),
    ('10000000-0000-0000-0000-000000000004', 'Add user profile page', 'Create a new user profile page with avatar upload functionality.', false, NOW() - INTERVAL '7 days', NOW() - INTERVAL '7 days'),
    ('10000000-0000-0000-0000-000000000005', 'Optimize database queries', 'Review and optimize slow database queries in the ticket repository.', false, NOW() - INTERVAL '6 days', NOW() - INTERVAL '6 days'),
    ('10000000-0000-0000-0000-000000000006', 'Fix responsive design issues', 'Fix layout issues on mobile devices for the ticket list view.', true, NOW() - INTERVAL '5 days', NOW() - INTERVAL '4 days'),
    ('10000000-0000-0000-0000-000000000007', 'Add email notifications', 'Implement email notifications for ticket updates and assignments.', false, NOW() - INTERVAL '4 days', NOW() - INTERVAL '4 days'),
    ('10000000-0000-0000-0000-000000000008', 'Create unit tests for repositories', 'Write comprehensive unit tests for all repository methods.', false, NOW() - INTERVAL '3 days', NOW() - INTERVAL '3 days'),
    ('10000000-0000-0000-0000-000000000009', 'Implement search functionality', 'Add full-text search capability for tickets and tags.', false, NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days'),
    ('10000000-0000-0000-0000-000000000010', 'Fix CORS configuration', 'Update CORS settings to allow requests from frontend domain.', true, NOW() - INTERVAL '1 day', NOW() - INTERVAL '12 hours'),
    ('10000000-0000-0000-0000-000000000011', 'Add ticket priority levels', 'Implement priority system with high, medium, and low levels.', false, NOW() - INTERVAL '12 hours', NOW() - INTERVAL '12 hours'),
    ('10000000-0000-0000-0000-000000000012', 'Create admin dashboard', 'Build admin dashboard for managing users and system settings.', false, NOW() - INTERVAL '11 hours', NOW() - INTERVAL '11 hours'),
    ('10000000-0000-0000-0000-000000000013', 'Implement file upload feature', 'Add ability to attach files to tickets.', false, NOW() - INTERVAL '10 hours', NOW() - INTERVAL '10 hours'),
    ('10000000-0000-0000-0000-000000000014', 'Fix memory leak in backend', 'Investigate and fix memory leak causing server crashes.', true, NOW() - INTERVAL '9 hours', NOW() - INTERVAL '8 hours'),
    ('10000000-0000-0000-0000-000000000015', 'Add ticket comments feature', 'Allow users to add comments to tickets for collaboration.', false, NOW() - INTERVAL '8 hours', NOW() - INTERVAL '8 hours'),
    ('10000000-0000-0000-0000-000000000016', 'Update dependencies', 'Update all npm and cargo dependencies to latest versions.', false, NOW() - INTERVAL '7 hours', NOW() - INTERVAL '7 hours'),
    ('10000000-0000-0000-0000-000000000017', 'Improve error handling', 'Enhance error messages and add proper error logging.', false, NOW() - INTERVAL '6 hours', NOW() - INTERVAL '6 hours'),
    ('10000000-0000-0000-0000-000000000018', 'Add ticket due dates', 'Implement due date functionality with reminders.', false, NOW() - INTERVAL '5 hours', NOW() - INTERVAL '5 hours'),
    ('10000000-0000-0000-0000-000000000019', 'Create API rate limiting', 'Add rate limiting to prevent API abuse.', true, NOW() - INTERVAL '4 hours', NOW() - INTERVAL '3 hours'),
    ('10000000-0000-0000-0000-000000000020', 'Fix tag color picker', 'Update tag color picker to use better color palette.', false, NOW() - INTERVAL '3 hours', NOW() - INTERVAL '3 hours'),
    ('10000000-0000-0000-0000-000000000021', 'Add ticket templates', 'Create ticket templates for common issue types.', false, NOW() - INTERVAL '2 hours', NOW() - INTERVAL '2 hours'),
    ('10000000-0000-0000-0000-000000000022', 'Implement ticket filtering', 'Add advanced filtering options for tickets.', false, NOW() - INTERVAL '1 hour', NOW() - INTERVAL '1 hour'),
    ('10000000-0000-0000-0000-000000000023', 'Add export functionality', 'Allow users to export tickets to CSV or JSON format.', false, NOW() - INTERVAL '50 minutes', NOW() - INTERVAL '50 minutes'),
    ('10000000-0000-0000-0000-000000000024', 'Fix pagination bug', 'Fix issue where pagination does not work correctly with filters.', true, NOW() - INTERVAL '40 minutes', NOW() - INTERVAL '30 minutes'),
    ('10000000-0000-0000-0000-000000000025', 'Create user guide', 'Write comprehensive user guide for the application.', false, NOW() - INTERVAL '30 minutes', NOW() - INTERVAL '30 minutes'),
    ('10000000-0000-0000-0000-000000000026', 'Add keyboard shortcuts', 'Implement keyboard shortcuts for common actions.', false, NOW() - INTERVAL '20 minutes', NOW() - INTERVAL '20 minutes'),
    ('10000000-0000-0000-0000-000000000027', 'Fix tag deletion cascade', 'Ensure tags are properly removed from tickets when deleted.', true, NOW() - INTERVAL '15 minutes', NOW() - INTERVAL '10 minutes'),
    ('10000000-0000-0000-0000-000000000028', 'Add ticket statistics', 'Display statistics about tickets on dashboard.', false, NOW() - INTERVAL '10 minutes', NOW() - INTERVAL '10 minutes'),
    ('10000000-0000-0000-0000-000000000029', 'Implement ticket assignments', 'Allow tickets to be assigned to specific users.', false, NOW() - INTERVAL '5 minutes', NOW() - INTERVAL '5 minutes'),
    ('10000000-0000-0000-0000-000000000030', 'Add ticket watchers', 'Allow users to watch tickets for updates.', false, NOW() - INTERVAL '4 minutes', NOW() - INTERVAL '4 minutes'),
    ('10000000-0000-0000-0000-000000000031', 'Fix timezone handling', 'Ensure all timestamps are displayed in user timezone.', false, NOW() - INTERVAL '3 minutes', NOW() - INTERVAL '3 minutes'),
    ('10000000-0000-0000-0000-000000000032', 'Add ticket history', 'Track and display history of ticket changes.', false, NOW() - INTERVAL '2 minutes', NOW() - INTERVAL '2 minutes'),
    ('10000000-0000-0000-0000-000000000033', 'Implement ticket cloning', 'Allow users to clone existing tickets.', false, NOW() - INTERVAL '1 minute', NOW() - INTERVAL '1 minute'),
    ('10000000-0000-0000-0000-000000000034', 'Add bulk operations', 'Enable bulk edit and delete operations for tickets.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000035', 'Fix validation errors', 'Improve form validation error messages.', true, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000036', 'Add ticket attachments preview', 'Show preview of attached files in ticket view.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000037', 'Implement ticket tags autocomplete', 'Add autocomplete when adding tags to tickets.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000038', 'Add ticket sorting options', 'Allow users to sort tickets by various fields.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000039', 'Fix search highlighting', 'Highlight search terms in ticket results.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000040', 'Add ticket quick actions', 'Implement quick action buttons for common operations.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000041', 'Create ticket reports', 'Generate reports for ticket statistics and trends.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000042', 'Add ticket notifications', 'Send browser notifications for ticket updates.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000043', 'Implement ticket workflows', 'Create custom workflows for ticket states.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000044', 'Fix performance issues', 'Optimize rendering performance for large ticket lists.', true, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000045', 'Add ticket archiving', 'Allow tickets to be archived instead of deleted.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000046', 'Implement ticket dependencies', 'Link tickets together to show dependencies.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000047', 'Add ticket time tracking', 'Track time spent on tickets.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000048', 'Create ticket kanban board', 'Implement kanban board view for tickets.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000049', 'Add ticket calendar view', 'Display tickets in calendar format.', false, NOW(), NOW()),
    ('10000000-0000-0000-0000-000000000050', 'Implement ticket AI suggestions', 'Use AI to suggest tags and similar tickets.', false, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Associate tags with tickets
-- Mix of tickets with different numbers of tags
INSERT INTO ticket_tags (ticket_id, tag_id) VALUES
    -- Ticket 1: urgent + bug
    ('10000000-0000-0000-0000-000000000001', '11111111-1111-1111-1111-111111111111'),
    ('10000000-0000-0000-0000-000000000001', '22222222-2222-2222-2222-222222222222'),
    -- Ticket 2: feature + frontend
    ('10000000-0000-0000-0000-000000000002', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000002', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 3: documentation
    ('10000000-0000-0000-0000-000000000003', '44444444-4444-4444-4444-444444444444'),
    -- Ticket 4: feature + frontend + enhancement
    ('10000000-0000-0000-0000-000000000004', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000004', '99999999-9999-9999-9999-999999999999'),
    ('10000000-0000-0000-0000-000000000004', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 5: backend + enhancement
    ('10000000-0000-0000-0000-000000000005', '88888888-8888-8888-8888-888888888888'),
    ('10000000-0000-0000-0000-000000000005', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 6: bug + frontend
    ('10000000-0000-0000-0000-000000000006', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000006', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 7: feature + backend
    ('10000000-0000-0000-0000-000000000007', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000007', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 8: testing + backend
    ('10000000-0000-0000-0000-000000000008', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
    ('10000000-0000-0000-0000-000000000008', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 9: feature + enhancement
    ('10000000-0000-0000-0000-000000000009', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000009', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 10: bug + urgent
    ('10000000-0000-0000-0000-000000000010', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000010', '11111111-1111-1111-1111-111111111111'),
    -- Ticket 11: feature + enhancement
    ('10000000-0000-0000-0000-000000000011', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000011', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 12: feature + backend
    ('10000000-0000-0000-0000-000000000012', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000012', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 13: feature + frontend
    ('10000000-0000-0000-0000-000000000013', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000013', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 14: bug + urgent + backend
    ('10000000-0000-0000-0000-000000000014', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000014', '11111111-1111-1111-1111-111111111111'),
    ('10000000-0000-0000-0000-000000000014', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 15: feature + enhancement
    ('10000000-0000-0000-0000-000000000015', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000015', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 16: documentation
    ('10000000-0000-0000-0000-000000000016', '44444444-4444-4444-4444-444444444444'),
    -- Ticket 17: bug + backend
    ('10000000-0000-0000-0000-000000000017', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000017', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 18: feature + enhancement
    ('10000000-0000-0000-0000-000000000018', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000018', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 19: backend + enhancement
    ('10000000-0000-0000-0000-000000000019', '88888888-8888-8888-8888-888888888888'),
    ('10000000-0000-0000-0000-000000000019', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 20: bug + frontend
    ('10000000-0000-0000-0000-000000000020', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000020', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 21: feature + enhancement
    ('10000000-0000-0000-0000-000000000021', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000021', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 22: feature + enhancement
    ('10000000-0000-0000-0000-000000000022', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000022', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 23: feature + enhancement
    ('10000000-0000-0000-0000-000000000023', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000023', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 24: bug + urgent
    ('10000000-0000-0000-0000-000000000024', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000024', '11111111-1111-1111-1111-111111111111'),
    -- Ticket 25: documentation
    ('10000000-0000-0000-0000-000000000025', '44444444-4444-4444-4444-444444444444'),
    -- Ticket 26: feature + frontend + enhancement
    ('10000000-0000-0000-0000-000000000026', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000026', '99999999-9999-9999-9999-999999999999'),
    ('10000000-0000-0000-0000-000000000026', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 27: bug + urgent
    ('10000000-0000-0000-0000-000000000027', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000027', '11111111-1111-1111-1111-111111111111'),
    -- Ticket 28: feature + enhancement
    ('10000000-0000-0000-0000-000000000028', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000028', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 29: feature + backend
    ('10000000-0000-0000-0000-000000000029', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000029', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 30: feature + enhancement
    ('10000000-0000-0000-0000-000000000030', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000030', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 31: bug + frontend
    ('10000000-0000-0000-0000-000000000031', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000031', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 32: feature + enhancement
    ('10000000-0000-0000-0000-000000000032', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000032', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 33: feature + enhancement
    ('10000000-0000-0000-0000-000000000033', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000033', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 34: feature + enhancement
    ('10000000-0000-0000-0000-000000000034', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000034', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 35: bug + urgent
    ('10000000-0000-0000-0000-000000000035', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000035', '11111111-1111-1111-1111-111111111111'),
    -- Ticket 36: feature + frontend
    ('10000000-0000-0000-0000-000000000036', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000036', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 37: feature + enhancement
    ('10000000-0000-0000-0000-000000000037', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000037', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 38: feature + enhancement
    ('10000000-0000-0000-0000-000000000038', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000038', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 39: feature + enhancement
    ('10000000-0000-0000-0000-000000000039', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000039', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 40: feature + frontend + enhancement
    ('10000000-0000-0000-0000-000000000040', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000040', '99999999-9999-9999-9999-999999999999'),
    ('10000000-0000-0000-0000-000000000040', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 41: feature + enhancement
    ('10000000-0000-0000-0000-000000000041', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000041', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 42: feature + frontend
    ('10000000-0000-0000-0000-000000000042', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000042', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 43: feature + backend
    ('10000000-0000-0000-0000-000000000043', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000043', '88888888-8888-8888-8888-888888888888'),
    -- Ticket 44: bug + urgent + frontend
    ('10000000-0000-0000-0000-000000000044', '22222222-2222-2222-2222-222222222222'),
    ('10000000-0000-0000-0000-000000000044', '11111111-1111-1111-1111-111111111111'),
    ('10000000-0000-0000-0000-000000000044', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 45: feature + enhancement
    ('10000000-0000-0000-0000-000000000045', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000045', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 46: feature + enhancement
    ('10000000-0000-0000-0000-000000000046', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000046', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 47: feature + enhancement
    ('10000000-0000-0000-0000-000000000047', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000047', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 48: feature + frontend + enhancement
    ('10000000-0000-0000-0000-000000000048', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000048', '99999999-9999-9999-9999-999999999999'),
    ('10000000-0000-0000-0000-000000000048', '55555555-5555-5555-5555-555555555555'),
    -- Ticket 49: feature + frontend
    ('10000000-0000-0000-0000-000000000049', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000049', '99999999-9999-9999-9999-999999999999'),
    -- Ticket 50: feature + enhancement
    ('10000000-0000-0000-0000-000000000050', '33333333-3333-3333-3333-333333333333'),
    ('10000000-0000-0000-0000-000000000050', '55555555-5555-5555-5555-555555555555')
ON CONFLICT DO NOTHING;
