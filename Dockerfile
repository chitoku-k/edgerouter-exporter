FROM golang:1.16.5-buster as build
WORKDIR /usr/src
COPY . /usr/src
RUN CGO_ENABLED=0 GOOS=linux GOARCH=mipsle go build -ldflags='-s -w'

FROM scratch
ENV GIN_MODE release
ENV PORT 80
COPY --from=build /usr/src/edgerouter-exporter /edgerouter-exporter
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
EXPOSE 80
CMD ["/edgerouter-exporter"]
