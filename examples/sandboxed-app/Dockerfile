FROM node:18-alpine

WORKDIR /app

# Copy package.json and package-lock.json
COPY package*.json ./

# Install dependencies
RUN npm install

# Copy application code
COPY . .

# Set environment variables (these will be overridden by Docker run command)
ENV DATAFOLD_API_HOST=datafold-api
ENV DATAFOLD_API_PORT=8080

# Command to run the application
CMD ["node", "app.js"]
