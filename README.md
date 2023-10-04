# Retrieval Augmented Generation API

This project aims to provide a powerful backend for a RESTful API that serves as a helpful assistant capable of retrieving relevant information from arbitrary databases. While it is geared towards learning, it can also be a valuable tool for accountants, researchers, and professionals across various domains who require accurate, digestible information quickly and efficiently.


# API
- `/query` supports a single question.

    ```bash
    curl -X POST http://0.0.0.0:5000/query -d '"What is the meaning of life?"' -H "Content-Type: application/json"
    ```
- `/conversation`
    ```bash
    curl -X POST "https://text-gen-webui.semanticallyinvalid.net/conversation" -H "Content-Type: application/json" -d '[{"User":"Why is it so difficult to put humans on Mars?"}]'
    ```