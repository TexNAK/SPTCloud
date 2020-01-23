FROM docker.pkg.github.com/texnak/science-paper-template/pandoc-build:latest

RUN mkdir -p /project
WORKDIR /project

COPY server-build.sh /

ENTRYPOINT ["/server-build.sh"]
CMD [""]