FROM node:10.9.0-alpine

WORKDIR /usr/src/app
COPY package*.json ./
RUN npm ci

COPY . .
ENV NODE_ENV "container"
EXPOSE 3009

CMD [ "npm", "run", "container" ]