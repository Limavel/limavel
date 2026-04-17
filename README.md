# Limavel 🍋‍🟩

> [!WARNING]  
> This project is in early development. Use at your own risk.

A Laravel Homestead replacement for macOS.


## Motivation

Laravel offers a great development environment based on containers for macOS with [Laravel Herd](https://herd.laravel.com/); however, some developers require a development environment that must be close to the production one, or simply need extra customization and security. Limavel offers the possibility of using virtualized development environments like [Laravel Homestead](https://laravel.com/docs/13.x/homestead) but without using Vagrant and Parallels Desktop.


## How it works

Limavel uses [Lima](https://lima-vm.io/) with the native [Apple Virtualization Framework](https://developer.apple.com/documentation/virtualization) and [Ansible](https://ansible.com) under the hood. It helps to create, edit, and provision virtual machines in macOS that are oriented to web development and especially to Laravel app development.

In order to use Limavel you need to previously install [Lima 2.x](https://formulae.brew.sh/formula/lima).

### Getting started

To use Limavel, follow the steps below:
1. Initialize configuration: ```limavel init```
2. Edit the configuration file (limavel.yaml)
3. Start the virtual machine: ```limavel start```

After the first start, Limavel will create a new virtual machine and run the provisioning process.


### Provisioning

You can run ```limavel provision``` for applying changes to the virtual machine related to software installation and configurations.

The provisioning process is based on Ansible playbooks.


### Playbooks customization

Limavel was developed for using Debian Trixie as a base; however, it's possible to customize the bootstrap scripts and playbooks to use other distributions.

To publish the OS bootstrap and Ansible playbook files in your filesystem, you can run ```limavel publish```. Feel free to modify these files for custom needs.

In the future I am planning to create a custom git repository for storing the playbook customizations from users.


### SSH Access

With the ```limavel ssh``` command you can access the virtual machine using SSH. The default system user is ```limavel``` and the password is ```secret```.


## Commands list

| Command                 | Description                                    |
|-------------------------|------------------------------------------------|
| ```init```      | Initialize the configuration file              |
| ```edit```      | Edit the configuration file                    |
| ```provision``` | Run the provisioning process                   |
| ```publish```   | Publish the OS bootstrap and Ansible playbooks |
| ```start```     | Start the virtual machine                      |
| ```stop```      | Stop the virtual machine                       |
| ```status```    | Check the status of the virtual machine        |
| ```ssh```       | Access the virtual machine using SSH           |
