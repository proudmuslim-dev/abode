#!/bin/bash

case $1 in
  create)
    echo "Creating databases..."

    diesel migration run --config-file ./diesel-app.toml --database-url ./app.db
    diesel migration run --config-file ./diesel-pending.toml --database-url ./pending.db
    ;;

  redo)
    echo "Redoing migrations..."

    diesel migration redo --config-file ./diesel-app.toml --database-url ./app.db
    diesel migration redo --config-file ./diesel-pending.toml --database-url ./pending.db
    ;;

  backup)
    echo "Creating backup..."

    BACKUP_PATH="backups/backup-$(date --iso-8601=seconds)"

    mkdir -p "$BACKUP_PATH"
    cp ./*.db "$BACKUP_PATH"/
    ;;

  setup-dev)
    USERNAME="value"
    PASSWORD="test123"

    echo "Creating dev admin account with username $USERNAME and password $PASSWORD..."

    echo "INSERT INTO users (id, username, password, admin) VALUES ('$(uuidgen)', '$USERNAME', '$PASSWORD', 1);" | sqlite3 app.db
    ;;

  *)
    echo "Please pass either 'create' or 'backup'"
    exit 1 
esac

echo "Done."
