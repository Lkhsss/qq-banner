CREATE TABLE "manager" (
    "name" TEXT NOT NULL,
    "password" TEXT NOT NULL,
    PRIMARY KEY ("name")
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_manager_by_name" ON "manager" ("name");