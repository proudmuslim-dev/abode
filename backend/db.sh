#!/bin/bash

case $1 in 
  push|p)
    echo "Pushing schema..."

    cargo prisma db push
    ;;

  generate|g)
    echo "Generating schema..."

    cargo prisma generate
    ;;

  generate_and_push|gp)
    ./db.sh push && ./db.sh generate
    ;;

  backup|b)
    echo "Not yet implemented!" && exit 1
    ;;

  *)
    echo "No arguments supplied."
    exit 1 
esac
