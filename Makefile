service:
	chmod +x club_cooee_trade
	cp clubcooee_trade.service /etc/systemd/system/
	systemctl daemon-reload
	systemctl start clubcooee_trade.service
	systemctl status clubcooee_trade.service

journal:
	journalctl -u clubcooee_trade.service -f

stop:
	systemctl stop clubcooee_trade.service

start:
	systemctl start clubcooee_trade.service

dependencies:
	curl -fsSL https://get.docker.com | sh

database:
	docker compose up mongodb -d

create_env:
	mv env.template .env
	echo "MONGODB_DSN=mongodb://mtz:7b36a5a4350a563bf6ca158386ebe3e7@localhost:27012" > .env

install: dependencies database service journal