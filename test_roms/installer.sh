#! /bin/bash

set -e

if ! [ $1 -a -r $1  ]; then
    echo "USAGE: $0 <data_file>"
    exit
fi

handle_installation() {
    local link=$1
    local install_type=$2
    local dest_rename=$3

    case $install_type in
        unzip_make_folder)
            wget -O $dest_rename.zip $link
            unzip $dest_rename.zip -d $dest_rename
            rm $dest_rename.zip
            ;;
        git)
            git clone $link $dest_rename
            ;;
        git_make)
            git clone $link $dest_rename
            cd $dest_rename
            make
            cd ..
            ;;
        git_make_install)
            git clone $link $dest_rename
            cd $dest_rename
            if [[ $4 ]]; then
                git checkout $4
            fi
            make
            sudo make install
            cd ..
            ;;
        none)
            wget -O $dest_rename $link
            ;;
        *)
            exit 1
            ;;
    esac
}

for line in `cat $1`; do
    if ! [[ $line =~ ^#.* ]]; then
        line=($(echo "$line" | tr ',' '\n'))
        if [ ${#line[@]} -lt 3 ];then
            exit 1
        fi
        handle_installation ${line[@]}
    fi
done

