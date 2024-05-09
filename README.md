# Retrieval Augmented Generation API

This project aims to provide a powerful backend for a RESTful API that serves as a helpful assistant capable of retrieving relevant information from arbitrary databases. While it is geared towards learning, it can also be a valuable tool for accountants, researchers, and professionals across various domains who require accurate, digestible information quickly and efficiently.

# ~~Quick~~start

0. **TODO** Memory leak: Most likely user error, but yet to find it. Ingest will require 350GB of RAM and or SWAP.

1. Pick a Wikipedia SQL DB `$mirror` from [https://meta.wikimedia.org/wiki/Mirroring_Wikimedia_project_XML_dumps#Current_Mirrors](https://meta.wikimedia.org/wiki/Mirroring_Wikimedia_project_XML_dumps#Current_Mirrors)

1. Get a dump from [https://`$mirror`/enwiki/YYYYMMDD/enwiki-YYYYMMDD-pages-articles.xml.bz2](https://$mirror/enwiki/YYYYMMDD/enwiki-YYYYMMDD-pages-articles.xml.bz2) and place it in `$working_directory` (`~/Documents/WIKIDUMPS/YYYYMMDD` in docker-compose.yaml)

1. `docker compose --profile triton --profile ingest --profile server up --build`

1. _Three hours later on an RTX 3090, with infinity embedding server and thenlper/gte-small_

1. You now have 2 sqlite files, index and document store. The index needs to be migrated to faiss and the document store (optionaly) needs to be moved to PostgreSQL.

   1. **Index:** `wikidex/convert_index.sh` will run a 'test' which will prepare the faiss index with a PCA factor of 128. This will take 30 - 60 minutes.
   1. **DocStore:**

      1. /tmp/migrate/migrate:
         ```lisp
         load database
           from sqlite:///db/wikipedia_docstore.sqlite
           into pgsql://wikidex:wikidex@0.0.0.0:5432/wikipedia
         with include drop, create tables, create indexes, reset sequences
         set work_mem to '1024MB', maintenance_work_mem to '1024MB';
         ```
         - sqlite:///db/wikipedia_docstore.sqlite is the path inside the docker container
         - pgsql://wikidex:wikidex@0.0.0.0:5432/wikipedia is the path to the external pgsql db
      1. ```bash
         docker run --rm -it \
           --volume ~/Documents/WIKIDUMPS/YYYYMMDD/docstore/:/db/ \
           --volume /tmp/migrate/migrate/:/commands/ \
           pgloader \
           pgloader \
           --dynamic-space-size 262144 \
           -v /commands/migrate
         ```

## Nvidia

### vllm

`docker compose --profile vllm --profile wikidex-local --profile server up --build`

### triton, batteries not included

`docker compose --profile triton --profile wikidex --profile server up --build`

## AMD

Unimplemented other than a stub in docker compose, but vllm and infinity _do_ support ROCm, and those are the only GPU Dependencies.

# API

- `/conversation`
  ```bash
  curl -X POST http://0.0.0.0:5000/conversation \
    -H "Content-Type: application/json" \
    -d '[{"User":"Why is it so difficult to put humans on Mars?"}]'
  ```
- `/streaming_conversation`
  ```bash
  curl -X POST https://0.0.0.0:5000/streaming_conversation \
    -H "Content-Type: application/json" \
    -d '{"messages": [{"User":"Why is it so difficult to put humans on Mars?"}]}'
  ```

## Documentation

- `/api-doc`
