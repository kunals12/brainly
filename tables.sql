CREATE SCHEMA `brainly`;

CREATE TABLE `brainly`.`users`(
    `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `username` VARCHAR(50) UNIQUE NOT NULL,
    `password` VARCHAR(256) NOT NULL
);

CREATE TABLE `brainly`.`contents`(
    `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `link` VARCHAR(256) NOT NULL,
    `type_` INT NOT NULL,
    `title` VARCHAR(256) NOT NULL,
    `user_id` INT NOT NULL,
    FOREIGN KEY (`user_id`) REFERENCES `brainly`.`users`(`id`) ON DELETE CASCADE
)