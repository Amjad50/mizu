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
        unzip_rename_inner_if_alone)
            wget -O $dest_rename.zip $link
            unzip -o $dest_rename.zip -d $dest_rename
            folder_elements=($(ls $dest_rename))
            if [[ ${#folder_elements[@]} -eq 1 ]]; then
                mv $dest_rename/${folder_elements[0]} $dest_rename_${folder_elements[0]}
                rm $dest_rename -r
                mv $dest_rename_${folder_elements[0]} $dest_rename
            else
                echo "Could not rename the inner folder since it contain many folders"
            fi
            rm $dest_rename.zip
            ;;
        unzip_make_folder)
            wget -O $dest_rename.zip $link
            unzip -o $dest_rename.zip -d $dest_rename
            rm $dest_rename.zip
            ;;
        git)
            if ! git clone $link $dest_rename && [ -d $dest_rename ]; then
                cd $dest_rename
                git pull
                cd ..
            fi
            ;;
        git_make)
            if ! git clone $link $dest_rename && [ -d $dest_rename ]; then
                cd $dest_rename
                git pull
                cd ..
            fi
            cd $dest_rename
            make
            cd ..
            ;;
        git_make_install)
            if ! git clone $link $dest_rename && [ -d $dest_rename ]; then
                echo "Already cloned"
            fi
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

