CREATE TABLE "users" (
    "id" INTEGER NOT NULL,
    "time" INTEGER NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_users_by_id" ON "users" ("id");