#!/bin/bash

case $1 in
  create)
    echo "Creating databases..."

    diesel migration run --config-file ./diesel-app.toml --database-url ./app.db
    diesel migration run --config-file ./diesel-pending.toml --database-url ./pending.db

    echo "Done."
    ;;

  backup)
    echo "Creating backup..."

    BACKUP_PATH="backups/backup-$(date --iso-8601=seconds)"

    mkdir -p "$BACKUP_PATH"
    cp ./*.db "$BACKUP_PATH"/
    echo "Done."
    ;;

  *)
    echo "Please pass either 'create' or 'backup'"
    exit 1 
esac
