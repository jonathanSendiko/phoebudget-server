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
To update the application after pushing new code:
```bash
git pull
docker-compose -f docker-compose.prod.yml build api
docker-compose -f docker-compose.prod.yml up -d --no-deps api
```

## Troubleshooting
- **SSL Fails**: Ensure port 80 is open on your server's firewall (`sudo ufw allow 80`).
- **DB Connection**: Check logs with `docker-compose -f docker-compose.prod.yml logs api`.
