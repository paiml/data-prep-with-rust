#!/usr/bin/env python3
"""Generate a synthetic, intentionally messy e-commerce orders CSV.

Used as the raw input for the bronze -> silver -> gold pipeline built in
Week 3 of the course. Messiness is injected on purpose: missing values,
duplicate rows, inconsistent casing/whitespace, a fully-empty column, mixed
date formats, and a few outlier numeric values.
"""

import csv
import random

random.seed(7)

REGIONS = ["North America", "Europe", "Asia", "South America", "Africa", "Oceania"]
REGION_CASINGS = lambda r: random.choice([r, r.lower(), r.upper(), f" {r} ", f"{r}  "])
CATEGORIES = ["Electronics", "Clothing", "Home", "Toys", "Books", "Sports"]
STATUSES = ["completed", "pending", "cancelled", "refunded"]
DATE_FORMATS = ["%Y-%m-%d", "%m/%d/%Y", "%d-%m-%Y"]


def random_date():
    import datetime

    start = datetime.date(2024, 1, 1)
    end = datetime.date(2025, 12, 31)
    delta = (end - start).days
    d = start + datetime.timedelta(days=random.randint(0, delta))
    fmt = random.choice(DATE_FORMATS)
    return d.strftime(fmt)


def make_row(order_id):
    unit_price = round(random.uniform(5, 500), 2)
    quantity = random.randint(1, 10)
    if random.random() < 0.01:
        quantity = random.choice([500, 1000])  # outlier
    return {
        "order_id": order_id,
        "customer_email": f"customer{order_id}@example.com",
        "region": REGION_CASINGS(random.choice(REGIONS)),
        "category": random.choice(CATEGORIES),
        "unit_price": unit_price,
        "quantity": quantity,
        "order_date": random_date(),
        "status": random.choice(STATUSES),
        "notes": "",  # fully-empty column
    }


def main():
    rows = []
    n = 600
    for i in range(1, n + 1):
        rows.append(make_row(1000 + i))

    # Inject missing values across a few columns.
    for _ in range(45):
        row = random.choice(rows)
        field = random.choice(["region", "category", "unit_price", "quantity", "order_date"])
        row[field] = ""

    # Inject duplicate rows (exact copies appended at the end).
    duplicates = [dict(r) for r in random.sample(rows, 20)]
    rows.extend(duplicates)

    random.shuffle(rows)

    fieldnames = [
        "order_id",
        "customer_email",
        "region",
        "category",
        "unit_price",
        "quantity",
        "order_date",
        "status",
        "notes",
    ]
    with open("data/raw/orders.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    print(f"wrote {len(rows)} rows to data/raw/orders.csv")


if __name__ == "__main__":
    main()
