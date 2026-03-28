from fastapi import FastAPI
from pydantic import BaseModel
from sentence_transformers import SentenceTransformer

app = FastAPI(title="Spine Embedding Service")

model = None
MODEL_NAME = "nomic-ai/nomic-embed-text-v1.5"


class EmbedRequest(BaseModel):
    text: str


class EmbedBatchRequest(BaseModel):
    texts: list[str]


class EmbedResponse(BaseModel):
    embedding: list[float]


class EmbedBatchResponse(BaseModel):
    embeddings: list[list[float]]


@app.on_event("startup")
def load_model():
    global model
    model = SentenceTransformer(MODEL_NAME, trust_remote_code=True)


@app.get("/health")
def health():
    return {"status": "ok", "model": MODEL_NAME}


@app.post("/embed", response_model=EmbedResponse)
def embed(req: EmbedRequest):
    embedding = model.encode([req.text], normalize_embeddings=True)[0]
    return EmbedResponse(embedding=embedding.tolist())


@app.post("/embed-batch", response_model=EmbedBatchResponse)
def embed_batch(req: EmbedBatchRequest):
    if not req.texts:
        return EmbedBatchResponse(embeddings=[])
    embeddings = model.encode(req.texts, normalize_embeddings=True)
    return EmbedBatchResponse(embeddings=[e.tolist() for e in embeddings])
