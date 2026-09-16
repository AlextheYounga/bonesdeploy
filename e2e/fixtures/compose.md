Source Tree:

```txt
compose
|-- Dockerfile
|-- compose.override.yaml
|-- compose.yaml
`-- index.html
```

`Dockerfile`:

```dockerfile
FROM python:3.14-alpine
WORKDIR /app
COPY index.html .
HEALTHCHECK --interval=1s --timeout=1s --retries=10 CMD wget -q -O /dev/null http://127.0.0.1:8080/
CMD ["python", "-m", "http.server", "8080"]
```

`compose.yaml`:

```yaml
services:
  web:
    build: .
    ports:
      - "127.0.0.1:18080:8080"
    depends_on:
      database:
        condition: service_healthy
    networks: [frontend, backend]
  direct:
    build: .
    ports:
      - "18081:8080"
    networks: [frontend]
  database:
    image: postgres:18-alpine
    environment:
      POSTGRES_PASSWORD: compose-e2e
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 1s
      timeout: 1s
      retries: 20
    volumes:
      - database:/var/lib/postgresql/data
    networks: [backend]
  worker:
    image: alpine:3.22
    command: ["sh", "-c", "while true; do sleep 60; done"]
    networks: [backend]
  state:
    image: alpine:3.22
    command: ["sh", "-c", "test -s /data/marker || date > /data/marker; while true; do sleep 60; done"]
    volumes:
      - state:/data
networks:
  frontend:
  backend:
volumes:
  database:
  state:
```

`compose.override.yaml`:

```yaml
services:
  web:
    environment:
      RELEASE_CHANNEL: e2e
```

`index.html`:

```html
compose-v1
```
