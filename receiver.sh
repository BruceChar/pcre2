#!/bin/bash
echo "Press [CTRL + C] to stop..."

# set the timeout 0 to unlock the conection
nc -kluw 1 127.0.0.1 7878 
