FROM node:10.9.0-alpine

WORKDIR /usr/src/app
COPY package*.json ./
RUN npm ci

COPY . .
ENV NODE_ENV "development"
EXPOSE 3015

CMD [ "npm", "run", "container" ]