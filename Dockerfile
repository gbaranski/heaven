FROM golang:1-alpine as builder

WORKDIR /app

COPY go.* ./
RUN go mod download

COPY *.go ./

RUN go build -v -o heaven .

FROM golang:1-alpine

COPY --from=builder /app/heaven /app/heaven
CMD ["/app/heaven"]