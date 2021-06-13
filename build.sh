ABSPATH=$(cd "$(dirname "$0")"; pwd -P)

echo $ABSPATH

PROJECT_DIR=$ABSPATH
TRD_DIR=$HOME/src/lush/3rd
LIBS_DIR=$PROJECT_DIR/3rd/libs

mkdir -p $TRD_DIR

cd $TRD_DIR

wget https://musl.libc.org/releases/musl-1.2.2.tar.gz
wget http://www.lua.org/ftp/lua-5.4.3.tar.gz

tar zxvf musl-1.2.2.tar.gz
tar zxvf lua-5.4.3.tar.gz

cd musl-1.2.2

./configure --prefix=$LIBS_DIR
make -j8
make install

cd $TRD_DIR/lua-5.4.3

make LDFLAGS="-static" CC="$LIBS_DIR/bin/musl-gcc -std=gnu99" -j8

mkdir $PROJECT_DIR/bin
cp src/lua  $PROJECT_DIR/bin/
cp src/luac $PROJECT_DIR/bin/

cd $PROJECT_DIR

chmod +x lush
