# CircuitDeck factorio-metrics exporter (development only).
#
# Serves the graftorio3 .prom textfile written by the Factorio server at
# /metrics.sh for Prometheus to scrape. See exporter/serve.sh for details.

FROM busybox:1.37

# Copy the exporter script and httpd CGI configuration.
COPY exporter/httpd.conf /circuitdeck/exporter/httpd.conf
COPY exporter/serve.sh /circuitdeck/exporter/serve.sh

RUN chmod +x /circuitdeck/exporter/serve.sh \
    && mkdir -p /srv /data/script-output/graftorio3 \
    && chown 65534:65534 /srv

USER nobody

ENTRYPOINT ["/bin/sh", "/circuitdeck/exporter/serve.sh"]