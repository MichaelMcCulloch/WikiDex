import tornado.ioloop
import tornado.web
import json
import logging
import argparse
import time
from langchain.embeddings import HuggingFaceEmbeddings


parser = argparse.ArgumentParser(description="Start the Tornado server.")
parser.add_argument("--port", type=int, default=8888, help="Port to listen on.")
args = parser.parse_args()

# Initialize the HuggingFaceEmbeddings object once
docstore_in = "/home/michael/Development/wikirip/safe_space/docstore.sqlitedb"
embed_out = "/home/michael/Development/wikirip/embeddings.sqlitedb"
document_count = 45551463
model_name = "thenlper/gte-small"
model_kwargs = {"device": "cuda"}
encode_kwargs = {"normalize_embeddings": False, "device": "cuda:0", "batch_size": 512}

hf = HuggingFaceEmbeddings(
    model_name=model_name,
    model_kwargs=model_kwargs,
    encode_kwargs=encode_kwargs,
)


class EmbedHandler(tornado.web.RequestHandler):
    def post(self):
        # Get the JSON payload containing the list of sentences
        data = json.loads(self.request.body)
        sentences = data.get("sentences", [])
        t1 = time.time()
        # Generate embeddings using the shared hf object
        embeddings = hf.embed_documents(sentences)
        t2 = time.time()

        logging.info(f"{len(sentences)} took {t2 -t2} seconds")
        # Return the embeddings as JSON
        self.write(json.dumps({"embeddings": embeddings}))


def make_app():
    return tornado.web.Application(
        [
            (r"/embed", EmbedHandler),
        ]
    )


if __name__ == "__main__":
    app = make_app()
    app.listen(args.port)
    logging.basicConfig(level=logging.INFO)

    logging.info(f"Listening on :{args.port}/embed")
    tornado.ioloop.IOLoop.current().start()
