if [[ -z "$(docker ps --filter name=masker-mysql -q)" ]]; then
  docker run -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root -v /home/denis/opt/mysql:/var/lib64/mysql/:rw --name masker-mysql -d mysql:5.7 
  while ! mysqladmin ping -h"127.0.0.1" --silent; do sleep 1; done
  mysql -h 127.0.0.1 -u root -p'root' < ./test_data/mysqlsampledatabase.sql
else 
  docker start masker-mysql
fi
