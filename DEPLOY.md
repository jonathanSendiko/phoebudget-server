# Deploying Phoebudget Server

Follow these steps to deploy the server on your Ubuntu instance.

## Prerequisites
- Docker and Docker Compose installed on the server.
- Domain name pointing to the server's public IP.

## 1. Clone the Repository
```bash
git clone <your-repo-url>
cd phoebudget-server
```

## 2. Configure Environment Secrets
Create a `.env.prod` file in the root directory. You can copy the template below:

```bash
# .env.prod

# App Domain (No http/https)
DOMAIN_NAME=api.yourdomain.com

# Let's Encrypt Email
LETSENCRYPT_EMAIL=admin@yourdomain.com

# Auth Secrets
JWT_SECRET=super_secret_jwt_key_please_change
GOOGLE_CLIENT_ID=your_google_web_client_id
FORCE_PREMIUM_SUBSCRIPTIONS=false

# Postgres Credentials (CHANGE THESE!)
POSTGRES_USER=postgres
POSTGRES_PASSWORD=secure_production_password
POSTGRES_DB=phoebudget
```

## 3. Run the Setup Script
The `setup.sh` script will:
1.  Download necessary SSL parameters.
2.  Replace the placeholder domain in Nginx configs with your `DOMAIN_NAME`.
3.  Start Nginx to handle the ACME challenge.
4.  Run Certbot to get your SSL certificates.
5.  Launch the full application stack.

```bash
./setup.sh
```

## 4. Updates
To update the application after pushing new code (this handles server, scheduler, and worker):
```bash
git pull
docker compose --env-file .env.prod -f docker-compose.prod.yml run --rm migrate
docker compose --env-file .env.prod -f docker-compose.prod.yml build
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d
```
Note: Usage of `build api` or `up api` is deprecated as we now have multiple services (`api`, `scheduler`, `worker`) sharing the same image/build context. Just running `build` and `up -d` ensures everything is fresh.

> [!WARNING]
> The above approach causes **5-15 seconds of downtime** while containers restart.

### Zero-Downtime Rolling Update (Alternative)
For production with active traffic, use staged restarts:
```bash
# 1. Build images first (no downtime)
docker compose --env-file .env.prod -f docker-compose.prod.yml build

# 1.1 Run migrations before restarting services
docker compose --env-file .env.prod -f docker-compose.prod.yml run --rm migrate

# 2. Deploy worker first (least critical)
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --no-deps worker

# 3. Then scheduler
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --no-deps scheduler

# 4. Finally API (brief blip, but minimal)
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --no-deps api
```

> [!CAUTION]
> Never use `docker compose down --volumes` in production — this deletes all database data!

## Troubleshooting
- **SSL Fails**: Ensure port 80 is open on your server's firewall (`sudo ufw allow 80`).
- **DB Connection**: Check logs with `docker compose --env-file .env.prod -f docker-compose.prod.yml logs api`.
- **Scheduler Logs**: `docker compose --env-file .env.prod -f docker-compose.prod.yml logs -f scheduler`
- **Worker Logs**: `docker compose --env-file .env.prod -f docker-compose.prod.yml logs -f worker`

## Architecture
The application consists of 3 binaries:
| Service | Description |
|---------|-------------|
| `api` | Main REST API server |
| `scheduler` | Polls DB every hour for due subscriptions, pushes jobs to Redis |
| `worker` | Processes subscription jobs from Redis, creates transactions |
