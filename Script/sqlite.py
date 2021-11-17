import os
import sys
import file_
import sqlite3
#使用方法：python3 sqlite.py 退休文件夹路径 数据库路径
#例如：退休文件夹路径为：archive/main/old_deb0 数据库路径为：deb_information.db,则使用方法为：python3 sqlite.py archive/main/old_deb0 deb_information.db
#可能遇到的问题：未创建数据库或deb数据表中各项信息。
def main(args):
    if os.path.exists(args[0]) and os.path.exists(args[1]):#将退休的deb信息导入到SQLite数据库中
        path = args[0]     #退休文件夹路径
        db=args[1] #数据库路径，使用前需要生成数据库及数据表
        file_dict = {}
        con = sqlite3.connect(db)    #建立与数据库的联系
        cur = con.cursor()           
        # new_list=os.listdir(path)
        os.chdir(path)
        file_.get_filedict('Repository', file_dict)
        filedict = sorted(file_dict.items(), reverse=False)
        for key in filedict: #插入deb的name，version，cpu，md5，tree，路径
            cur.execute('INSERT INTO deb VALUES (?,?,?,?,?,?)',(key[0].split(sep='_')[0],key[0].split(sep='_')[1],key[0].split(sep='_')[2].split(sep='.')[0],file_.md5sum(key[1]),key[1],path[-1]))
            con.commit()
        con.close()

if __name__=='__main__':
    main(sys.argv[1:])
