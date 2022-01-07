FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

ARG UID=1000
ARG GID=1000

RUN apt update
RUN apt install -y sudo curl gcc
RUN apt install -y locales locales-all

RUN groupadd -o -r user -g $GID && useradd -r -ms /bin/bash -g $GID -u $UID user
RUN echo "user:user" | chpasswd
RUN adduser user sudo

USER user

ENV LC_ALL en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US.UTF-8
