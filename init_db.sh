#!/bin/bash

# Create the database
psql -U postgres -c "CREATE DATABASE dodo;"

# Run the migrations
sqlx database create
sqlx migrate run 