#!/bin/bash

# Exit on error
set -e

if [ ! -f .env.prod ]; then
    echo "❌ Error: .env.prod file not found!"
    echo "Please create one based on .env.example (or the provided template) before running this script."
    exit 1
fi

echo "🔍 Loading environment variables..."
export $(cat .env.prod | grep -v '#' | xargs)

domains=($DOMAIN_NAME)
rsa_key_size=4096
data_path="./certbot"
email="$LETSENCRYPT_EMAIL" # Can be empty if you don't want to provide email
staging=0 # Set to 1 if you're testing your setup to avoid hitting request limits

if [ -d "$data_path" ]; then
    read -p "Existing data found for $domains. Continue and replace existing certificate? (y/N) " decision
    if [ "$decision" != "Y" ] && [ "$decision" != "y" ]; then
        exit
    fi
fi

if [ ! -e "$data_path/conf/options-ssl-nginx.conf" ] || [ ! -e "$data_path/conf/ssl-dhparams.pem" ]; then
    echo "### Downloading recommended TLS parameters ..."
    mkdir -p "$data_path/conf"
    curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf > "$data_path/conf/options-ssl-nginx.conf"
    curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem > "$data_path/conf/ssl-dhparams.pem"
    echo
fi

echo "### Updating Nginx configuration with domain name..."
# We use sed to replace YOUR_DOMAIN.COM with the actual domain in .env.prod
sed -i "s/YOUR_DOMAIN.COM/$DOMAIN_NAME/g" ./nginx/conf.d/init.conf
sed -i "s/YOUR_DOMAIN.COM/$DOMAIN_NAME/g" ./nginx/conf.d/default.conf

echo "### Starting Nginx (HTTP only)..."
docker-compose -f docker-compose.prod.yml up --force-recreate -d nginx
echo

echo "### Requesting Let's Encrypt certificate for $domains ..."
# Join $domains to -d args
domain_args=""
for domain in "${domains[@]}"; do
    domain_args="$domain_args -d $domain"
done

# Select appropriate email arg
case "$email" in
  "") email_arg="--register-unsafely-without-email" ;;
  *) email_arg="--email $email" ;;
esac

# Enable staging mode if needed
if [ $staging != "0" ]; then staging_arg="--staging"; fi

docker compose -f docker-compose.prod.yml run --rm --entrypoint "\
  certbot certonly --webroot -w /var/www/certbot \
    $staging_arg \
    $email_arg \
    $domain_args \
    --rsa-key-size $rsa_key_size \
    --agree-tos \
    --force-renewal" certbot
echo

echo "### Reloading Nginx ..."
docker compose -f docker-compose.prod.yml exec nginx nginx -s reload

echo "### Preparing Production Configuration ..."
# Swap the init config (which only handles ACME) with the real config (which does SSL termination)
# We can just keep default.conf as the main one, but we need to ensure the user started with init.conf logic active
# For simplicity in this script:
# 1. We started with init.conf effectively if we just mapped it? 
# Actually, the docker-compose maps the whole conf.d. 
# We need to ensure only init.conf is active first, then switch.
# A simpler way: Rename default.conf.disabled to default.conf after cert is obtained.

mv ./nginx/conf.d/init.conf ./nginx/conf.d/init.conf.bak
# Ensure default.conf is active
docker compose -f docker-compose.prod.yml restart nginx

echo "### Starting entire stack ..."
docker compose -f docker-compose.prod.yml up -d

echo "✅ Setup complete! Your app should be live at https://$DOMAIN_NAME"
