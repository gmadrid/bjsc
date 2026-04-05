-- user_deck: stores each user's spaced-repetition deck state and study mode.
-- One row per user, upserted on conflict via user_id.
create table if not exists user_deck (
  id         uuid primary key default gen_random_uuid(),
  user_id    uuid not null unique references auth.users(id) on delete cascade,
  study_mode text not null default 'All',
  deck       jsonb not null default '{}'::jsonb,
  updated_at timestamptz not null default now()
);

-- answer_log: append-only log of every strategy answer the user submits.
create table if not exists answer_log (
  id              bigint generated always as identity primary key,
  user_id         uuid not null references auth.users(id) on delete cascade,
  table_index     text not null,
  correct         boolean not null,
  player_action   text not null,
  correct_action  text not null,
  created_at      timestamptz not null default now()
);

-- Index for the most common query pattern: fetching a user's recent logs.
create index if not exists idx_answer_log_user_created
  on answer_log (user_id, created_at desc);

-- Enable Row Level Security on both tables.
alter table user_deck enable row level security;
alter table answer_log enable row level security;

-- user_deck policies: users can only read/write their own row.
create policy "Users can select their own deck"
  on user_deck for select
  using (auth.uid() = user_id);

create policy "Users can insert their own deck"
  on user_deck for insert
  with check (auth.uid() = user_id);

create policy "Users can update their own deck"
  on user_deck for update
  using (auth.uid() = user_id)
  with check (auth.uid() = user_id);

-- answer_log policies: users can read and insert their own logs.
create policy "Users can select their own logs"
  on answer_log for select
  using (auth.uid() = user_id);

create policy "Users can insert their own logs"
  on answer_log for insert
  with check (auth.uid() = user_id);

-- Trigger to auto-update updated_at on user_deck changes.
create or replace function update_updated_at()
returns trigger as $$
begin
  new.updated_at = now();
  return new;
end;
$$ language plpgsql;

create trigger set_updated_at
  before update on user_deck
  for each row
  execute function update_updated_at();
