import tornado.ioloop
import tornado.web
import json
import logging
import argparse
from langchain.embeddings import HuggingFaceEmbeddings


parser = argparse.ArgumentParser(description="Start the Tornado server.")
parser.add_argument("--port", type=int, default=9000, help="Port to listen on.")
parser.add_argument("--model", type=str, default="thenlper/gte-small", help="Sentence Bert model to use.")
parser.add_argument("--batch", type=int, default=640, help="Maximum size of an embedding batch.")
args = parser.parse_args()

model_name = args.model
model_kwargs = {"device": "cuda"}
encode_kwargs = {"normalize_embeddings": False, "device": "cuda:0", "batch_size": args.batch}

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
        # Generate embeddings using the shared hf object
        embeddings = hf.embed_documents(sentences)

        # Return the embeddings as JSON
        self.write(json.dumps({"embeddings": embeddings}))


def make_app():
    return tornado.web.Application(
        [
            (r"/", EmbedHandler),
        ]
    )


if __name__ == "__main__":
    app = make_app()

    app.listen(args.port, max_buffer_size=None)

    logging.basicConfig(level=logging.INFO)

    logging.info(f"Listening on :{args.port}/")
    tornado.ioloop.IOLoop.current().start()
