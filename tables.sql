CREATE SCHEMA `brainly`;

CREATE TABLE `brainly`.`users`(
    `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `username` VARCHAR(50) UNIQUE NOT NULL,
    `password` VARCHAR(256) NOT NULL
);
