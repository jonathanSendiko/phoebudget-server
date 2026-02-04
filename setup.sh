#!/usr/bin/env bash

set -Eeuo pipefail

ENV_FILE=".env.prod"
COMPOSE_FILE="docker-compose.prod.yml"
NGINX_CONF_DIR="./nginx/conf.d"
INIT_CONF="${NGINX_CONF_DIR}/init.conf"
DEFAULT_CONF="${NGINX_CONF_DIR}/default.conf"
DEFAULT_DISABLED_CONF="${NGINX_CONF_DIR}/default.conf.disabled"
CERTBOT_PATH="./certbot"
rsa_key_size=4096

read_env_var() {
    local key="$1"
    grep -E "^${key}=" "$ENV_FILE" | tail -n 1 | cut -d'=' -f2- | tr -d '"' | tr -d "'" | tr -d '\r' || true
}

if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Error: $ENV_FILE file not found."
    exit 1
fi

echo "🔍 Loading deployment configuration..."
DOMAIN_NAME="$(read_env_var "DOMAIN_NAME")"
LETSENCRYPT_EMAIL="$(read_env_var "LETSENCRYPT_EMAIL")"
LETSENCRYPT_STAGING="$(read_env_var "LETSENCRYPT_STAGING")"

if [ -z "$DOMAIN_NAME" ]; then
    echo "❌ Error: DOMAIN_NAME is missing in $ENV_FILE"
    exit 1
fi

domain_input="${DOMAIN_NAME//,/ }"
read -r -a domains <<< "$domain_input"
if [ "${#domains[@]}" -eq 0 ]; then
    echo "❌ Error: DOMAIN_NAME must include at least one domain."
    exit 1
fi

primary_domain="${domains[0]}"
server_names="${domains[*]}"

email_args=(--register-unsafely-without-email)
if [ -n "$LETSENCRYPT_EMAIL" ]; then
    email_args=(--email "$LETSENCRYPT_EMAIL")
fi

staging_args=()
if [ "${LETSENCRYPT_STAGING:-0}" = "1" ]; then
    staging_args=(--staging)
fi

domain_args=()
for domain in "${domains[@]}"; do
    domain_args+=("-d" "$domain")
done

echo "ℹ️  Migration mode reminder:"
echo "   - Point DNS A/AAAA records to this new server before running certbot."
echo "   - Using TTL=300 is good for cutover."
echo "   - Requested domains: ${server_names}"
echo

mkdir -p "$NGINX_CONF_DIR" "$CERTBOT_PATH/conf" "$CERTBOT_PATH/www"

# Recover from interrupted previous run if needed.
if [ -f "$DEFAULT_DISABLED_CONF" ]; then
    mv "$DEFAULT_DISABLED_CONF" "$DEFAULT_CONF"
fi

echo "### Rendering Nginx configuration for ${server_names} ..."
cat > "$INIT_CONF" <<EOF
server {
    listen 80;
    listen [::]:80;
    server_name ${server_names};

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://\$host\$request_uri;
    }
}
EOF

cat > "$DEFAULT_CONF" <<EOF
server {
    listen 80;
    listen [::]:80;
    server_name ${server_names};

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://\$host\$request_uri;
    }
}

server {
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name ${server_names};

    ssl_certificate /etc/letsencrypt/live/${primary_domain}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/${primary_domain}/privkey.pem;

    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    resolver 127.0.0.11 valid=10s ipv6=off;
    set \$upstream "api:3000";

    location / {
        proxy_pass http://\$upstream;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

if [ ! -e "$CERTBOT_PATH/conf/options-ssl-nginx.conf" ] || [ ! -e "$CERTBOT_PATH/conf/ssl-dhparams.pem" ]; then
    echo "### Downloading recommended TLS parameters ..."
    curl -s "https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf" > "$CERTBOT_PATH/conf/options-ssl-nginx.conf"
    curl -s "https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem" > "$CERTBOT_PATH/conf/ssl-dhparams.pem"
    echo
fi

echo "### Starting Nginx with ACME-only config ..."
mv "$DEFAULT_CONF" "$DEFAULT_DISABLED_CONF"
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up --force-recreate -d --no-deps nginx
echo

echo "### Requesting/refreshing certificate for: ${server_names} ..."
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" run --rm --entrypoint certbot certbot \
    certonly --webroot -w /var/www/certbot \
    "${staging_args[@]}" \
    "${email_args[@]}" \
    "${domain_args[@]}" \
    --rsa-key-size "$rsa_key_size" \
    --agree-tos \
    --non-interactive \
    --keep-until-expiring \
    --expand \
    --cert-name "$primary_domain"
echo

echo "### Enabling full HTTPS config ..."
rm -f "$INIT_CONF"
mv "$DEFAULT_DISABLED_CONF" "$DEFAULT_CONF"
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" restart nginx
echo

echo "### Starting entire production stack ..."
docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" up -d

echo "✅ Setup complete! App URL: https://${primary_domain}"
