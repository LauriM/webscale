FROM scorpil/rust
MAINTAINER Lauri Mäkinen

COPY . /root/

RUN apt-get update && apt-get install -y build-essential libssl-dev

RUN cd /root/ && cargo build --release

WORKDIR /root/
CMD ["cargo", "run", "--release"]
