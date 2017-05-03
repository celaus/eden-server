## -*- docker-image-name: "clma/eden-server" -*-
FROM greyltc/archlinux:latest
MAINTAINER clma <claus.matzinger+kb@gmail.com>

ENV EDEN_VER x86-latest
RUN pacman -Sy --noconfirm tar gzip  && pacman -Sc --noconfirm && rm -Rf /usr/share
RUN curl -s https://x5ff.xyz:8080/builds/eden-server-$EDEN_VER.tgz | tar xfz - && chmod +x /eden/eden-server
ENV PATH /eden:$PATH
EXPOSE 6200
WORKDIR /eden
CMD ["eden-server"]
