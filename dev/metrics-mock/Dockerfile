# Mock graftorio3-compatible metrics generator (development only).
FROM python:3.12-alpine

RUN addgroup -S metrics && adduser -S -G metrics mock

COPY serve.py /app/serve.py
RUN chmod +r /app/serve.py

USER mock
EXPOSE 9105

# Health check: the metrics endpoint must respond.
HEALTHCHECK --interval=15s --timeout=3s --retries=3 \
  CMD python3 -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:9105/metrics', timeout=2)" || exit 1

CMD ["python3", "/app/serve.py"]