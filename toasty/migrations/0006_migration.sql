ALTER TABLE "users" DROP COLUMN "report";
-- #[toasty::breakpoint]
CREATE TABLE "reporter" (
    "name" TEXT NOT NULL,
    "count" INTEGER NOT NULL,
    PRIMARY KEY ("name")
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_reporter_by_name" ON "reporter" ("name");