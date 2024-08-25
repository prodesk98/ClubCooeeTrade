service:
	chmod +x ClubCooee
	cp clubcooee.service /etc/systemd/system/
	systemctl daemon-reload
	systemctl start clubcooee.service
	systemctl status clubcooee.service

journal:
	journalctl -u clubcooee.service -f

stop:
	systemctl stop clubcooee.service

start:
	systemctl start clubcooee.service

dependencies:
	curl -fsSL https://get.docker.com | sh

database:
	docker compose up mongodb -d

create_env:
	mv env.template .env
	echo "MONGODB_DSN=mongodb://mtz:7b36a5a4350a563bf6ca158386ebe3e7@localhost:27012" > .env

install: dependencies database service journal