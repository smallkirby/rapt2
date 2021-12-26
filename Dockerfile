FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update
RUN apt install -y sudo curl gcc

RUN groupadd -r user && useradd -r -ms /bin/bash -g user user
RUN echo "user:user" | chpasswd
RUN adduser user sudo


USER user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
USER user
ENV PATH /home/user/.cargo/bin:$PATH

ADD . /home/user/rapt2
USER root
RUN /bin/chown user:user /home/user/rapt2
USER user
WORKDIR /home/user/rapt2
RUN cargo build
