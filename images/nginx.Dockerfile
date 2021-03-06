FROM nginx:alpine

COPY images/nginx/nginx.conf /etc/nginx/nginx.conf

RUN rm -rf /usr/share/nginx/html/*

EXPOSE 80

ENTRYPOINT ["nginx", "-g", "daemon off;"]